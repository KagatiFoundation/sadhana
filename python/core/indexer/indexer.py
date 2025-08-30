from dataclasses import dataclass
from bs4 import BeautifulSoup
from ..text import *
import sys

@dataclass
class IndexerOpts:
    pass

class Indexer:
    def __init__(self, opts: IndexerOpts, html: str):
        self.opts = opts
        self.html = html

    def index(self):
        self.prepare_data_for_indexing()
        return self.processed_title

    def prepare_data_for_indexing(self):
        bs = BeautifulSoup(self.html, 'html.parser')
        raw_title = bs.title

        if raw_title is None:
            print("No title is present on the page! Aborting indexing...")
            sys.exit(-1)

        self.processed_title = clean_and_lemmatize(str(raw_title.string))