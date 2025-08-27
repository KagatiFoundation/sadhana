if __name__ == "__main__":
    from ..core.crawler.engine import *
    crawl = Crawler(
        CrawlerOpts(
            max_depth=2,
            seed_url="https://developer.mozilla.org/en-US/docs/Web/JavaScript"
        )
    )

    crawl.crawl()