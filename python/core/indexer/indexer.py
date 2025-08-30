from dataclasses import dataclass
from bs4 import BeautifulSoup
from collections import Counter
import redis
import math

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

        self.db_handle: SadhanaDB = db_handle
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


    def index(self, html: str, doc_id: str):
        self.content = html
        self.doc_id = doc_id

        try:
            self.prepare_data_for_indexing()
            self.update_doc_freq(self.processed_title)
            self.update_doc_freq(self.processed_content)
            self.doc_count += 1
        except:
            return

        title_scores = self.tfidf(self.processed_title)
        content_scores = self.tfidf(self.processed_content)

        for word in self.processed_title + self.processed_content:
            rank_items = self.db_handle.get_rank_item(word)
            for item in rank_items:
                print(item)


    def update_doc_freq(self, terms: list[str]):
        unique_terms = set(terms)
        for t in unique_terms:
            self.doc_freq[t] = self.doc_freq.get(t, 0) + 1


    def tfidf(self, terms):
        tf_counts = Counter(terms)
        total_terms = len(terms)
        for term, count in tf_counts.items():
            tf = count / total_terms
            idf = math.log(self.doc_count + 1) / (1 + self.doc_freq[term])
            tf_idf = tf * idf
            self.inverted_index.setdefault(term, []).append((self.doc_id, tf_idf))