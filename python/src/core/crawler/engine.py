from . import http_req
from ..html import preprocess

from dataclasses import dataclass

@dataclass
class CrawlerOpts:
    max_depth: int = 0
    seed_url: str = ""
    follow_external_links: bool = False


class Crawler:
    def __init__(self, opts: CrawlerOpts):
        self.opts = opts
        self.link_queue = [(opts.seed_url, 0)]
        self.visited = set()


    def crawl(self):
        while self.link_queue:
            link_to_visit, depth = self.link_queue.pop(0)
            if link_to_visit in self.visited or (self.opts.max_depth and depth > self.opts.max_depth):
                continue
            
            self.visited.add(link_to_visit)

            html = http_req.fetch_html("")
            new_links = preprocess.extract_links(html)

            for new_link in new_links:
                if new_link not in self.visited:
                    self.link_queue.append((new_link, depth + 1))

        print(self.visited)