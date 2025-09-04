from dataclasses import dataclass
from bs4 import BeautifulSoup
from collections import Counter
import redis

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

        self.db_handle = SadhanaDB()
        self.redis_handle: SadhanaRedisMgr = redis_handle


    def prepare_data_for_indexing(self):
        bs = BeautifulSoup(self.content, 'html.parser')

        raw_title = bs.title
        if raw_title is None:
            raise NoTitleFoundException("Title not present on the document.")

        self.raw_title = str(raw_title.string)
        self.processed_title = clean_and_lemmatize(self.raw_title)

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
        self.db_handle.batch_insert_new_rank_items(all_terms, doc_id, self.raw_title)

    
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