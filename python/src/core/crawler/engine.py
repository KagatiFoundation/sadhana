from . import http_req

from dataclasses import dataclass
from collections import deque
import asyncio
from typing import Set, Deque

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

            tasks = [self.crawl_link(link) for link in batch]
            results = await asyncio.gather(*tasks)

            async with self.lock:
                for html in results:
                    print(html)

        depth += 1

    async def crawl_link(self, link: str):
        return await http_req.fetch_html(link)