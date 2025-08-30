import pytest
from src.core.crawler.engine import *
from src.core.crawler.http_req import *

@pytest.mark.asyncio
async def test_is_valid_url_and_same_domain():
    opts = CrawlerOpts(seed_url="http://example.com")
    crawler = Crawler(opts)

    assert crawler.is_valid_url("http://example.com/page")
    assert not crawler.is_valid_url("/relative/path")

    assert crawler.is_same_domain("http://example.com/other")
    assert not crawler.is_same_domain("http://another.com")


@pytest.mark.asyncio
async def test_crawl_link(monkeypatch):
    async def fake_fetch_html(url: str):
        return "<html><body><a href='/next'>next</a></body></html>"

    monkeypatch.setattr(http_req, "fetch_html", fake_fetch_html)

    opts = CrawlerOpts(seed_url="http://example.com")
    crawler = Crawler(opts)

    html = await crawler.crawl_link("http://example.com")
    assert "next" in html


@pytest.mark.asyncio
async def test_prepare_links_for_next_batch(monkeypatch):
    monkeypatch.setattr("src.core.html.preprocess.extract_links", lambda html: ["/page1", "/page2"])

    opts = CrawlerOpts(seed_url="http://example.com")
    crawler = Crawler(opts)

    html = '''
        <html>
            <a href='/page1'>Page 1</a>
            <a href='/page2'>Page 2</a>
        </html>
    '''
    crawler.prepare_links_for_next_batch(html, "http://example.com")

    queued = list(crawler.links_to_crawl)
    assert "page1" in queued or "/page1" in queued
    assert "page2" in queued or "/page2" in queued