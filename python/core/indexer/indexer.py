from dataclasses import dataclass
from bs4 import BeautifulSoup
from collections import Counter
import sys, redis
import pprint

from ..text import *
from ..db import SadhanaDB, SadhanaRedisMgr

'''
Tags which may contain text
'''
CONTENT_TAGS = set(['p', 'h1', 'h2', 'h3', 'h4', 'summary', 'article'])


class NoTitleFoundException(Exception):
    def __init__(self, msg: str):
        super.__init__(msg)


@dataclass
class IndexerOpts:
    pass

class Indexer:
    def __init__(self, opts: IndexerOpts, redis_handle: redis.Redis, db_handle: SadhanaDB):
        self.opts = opts
        self.doc_count = 0
        self.doc_freq = {}
        self.inverted_index = {}

        self.db_handle: dict[str, list[dict]] = {} # SadhanaDB()
        self.redis_handle: SadhanaRedisMgr = redis_handle


    def prepare_data_for_indexing(self):
        bs = BeautifulSoup(self.content, 'html.parser')

        raw_title = bs.title
        if raw_title is None:
            raise NoTitleFoundException("Title not present on the document.")

        self.processed_title = clean_and_lemmatize(str(raw_title.string))

        raw_content = ""
        for c_tag in CONTENT_TAGS:
            for content in bs.find_all(c_tag):
                raw_content += " " + str(content.string)
            
        self.processed_content = clean_and_lemmatize(str(raw_content))


    async def index(self, html: str, doc_id: str):
        self.content = html
        self.doc_id = doc_id

        try:
            self.prepare_data_for_indexing()
        except Exception as e:
            raise Exception(f"Cannot index '{doc_id}': {e}")

        all_terms = self.processed_title + self.processed_content
        self.update_term_freq(all_terms)
        rank_metric = await self.compute_tfidf(all_terms)

        for word in all_terms:
            terms_score = rank_metric.get(word)
            if terms_score is None:
                print("Impossible happened...")
                sys.exit(-1)

            new_rank_item = {
                "url": doc_id,
                "title": " ".join(self.processed_title),
                "score": terms_score
            }

            if word not in self.db_handle:
                self.db_handle[word] = [new_rank_item]
            else:
                if not any(item['url'] == doc_id for item in self.db_handle[word]):
                    self.db_handle[word].append(new_rank_item)

        self.redis_handle.incr_internal_value('doc-count')
        pprint.pprint(self.db_handle)

    
    async def compute_tfidf(self, terms: list[str]):
        term_counts = Counter(terms)
        total_terms = len(terms)

        total_docs = self.redis_handle.get_internal_value('doc-count')
        if total_docs is None:
            total_docs = 1
        else:
            total_docs = int(total_docs)

        tf_scores: dict[str, float] = {}

        for term, count in term_counts.items():
            term_freq = int(self.redis_handle.get_term_freq(term))
            tf_score = count / total_terms
            idf_score = total_docs / term_freq
            tf_scores[term] = tf_score * idf_score

        return tf_scores

    
    def update_term_freq(self, terms):
        for term in terms:
            self.redis_handle.store_term_freq(term, 1)