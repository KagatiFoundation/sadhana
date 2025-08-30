from bs4 import BeautifulSoup

def extract_links(html_str: str) -> str:
    parser = BeautifulSoup(html_str, 'html.parser')
    return parser.find_all("a")