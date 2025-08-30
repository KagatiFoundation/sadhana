from core.crawler.engine import *
import asyncio

async def main():
    opts = CrawlerOpts(max_depth=2, seed_url="https://example.com")
    engine = Crawler(opts)
    await engine.start_crawling()

try:
    asyncio.run(main())
except KeyboardInterrupt:
    print("Shutdown...")