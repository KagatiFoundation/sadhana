from . import http_req
from ..html.preprocess import *
from ..indexer.indexer import *
from ..db import SadhanaDB, SadhanaRedisMgr

from dataclasses import dataclass
from collections import deque
import asyncio
from typing import Set, Deque
import redis
from urllib.parse import urljoin, urlparse

@dataclass
class CrawlerOpts:
    max_depth: int = 0
    seed_url: str = ""
    follow_external_links: bool = False


class Crawler:
    def __init__(self, opts: CrawlerOpts):
        self.opts = opts
        self.visited: Set[str] = set()
        self.links_to_crawl: Deque[str] = deque()
        self.lock = asyncio.Lock()

        self.links_to_crawl.append(opts.seed_url)

        # database
        self.db_handle = SadhanaDB()

        # redis client
        self.redis_handle = SadhanaRedisMgr()

        # indexer
        self.indexer = Indexer(IndexerOpts(), self.redis_handle, self.db_handle)


    async def start_crawling(self):
        depth = 0
        while depth <= self.opts.max_depth:
            batch = []

            async with self.lock:
                while self.links_to_crawl:
                    link = self.links_to_crawl.popleft()
                    if link not in self.visited:
                        self.visited.add(link)
                        batch.append(link)

            for next_link in batch:
                crawl_task = self.crawl_link(next_link)
                crawled_data = await crawl_task
                
                # index
                self.indexer.index(crawled_data, next_link)
                self.prepare_links_for_next_batch(crawled_data, next_link)

            depth += 1


    async def crawl_link(self, link: str):
        return await http_req.fetch_html(link)


    def prepare_links_for_next_batch(self, html: str, base_url):
        new_links = extract_links(html)[:10]
        for link in new_links:
            abs_path = link if self.is_valid_url(link) else urljoin(base_url, link)

            if not self.opts.follow_external_links and not self.is_same_domain(abs_path):
                continue

            if abs_path not in self.visited:
                self.links_to_crawl.append(abs_path)


    def is_valid_url(self, url: str) -> bool:
        components = urlparse(url)
        return all([components.scheme != '', components.netloc != ''])


    def is_same_domain(self, url: str) -> bool:
        return urlparse(self.opts.seed_url).netloc == urlparse(url).netloc