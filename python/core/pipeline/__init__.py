from ..crawler.engine import CrawlerOpts, Crawler
from ..indexer import Indexer, IndexerOpts
from ..db import SadhanaDB, SadhanaRedisMgr

class EnginePipeline:
    def __init__(self):
        self.db_handle = SadhanaDB()
        self.redis_handle = SadhanaRedisMgr()
        self.crawler = Crawler(CrawlerOpts(max_depth=1, seed_url=""))
        self.indexer = Indexer(IndexerOpts(), self.redis_handle, self.db_handle)

    
    async def process_batch(self, urls: list[str]):
        self.batch = urls

        for url in self.batch:
            self.crawler.opts.seed_url = url
            async for crawled_data, link in self.crawler.crawl():
                await self.indexer.index(crawled_data, link)

    
    def __del__(self):
        self.db_handle.close()


__all__ = ['EnginePipeline']