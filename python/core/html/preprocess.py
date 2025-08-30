from bs4 import BeautifulSoup

def extract_links(html_str: str) -> str:
    parser = BeautifulSoup(html_str, 'html.parser')
    a_tags = parser.find_all('a', href=True)
    return [a_tag['href'] for a_tag in a_tags]

def extract_title(html: str) -> str:
    parser = BeautifulSoup(html, 'html.parser')
    return parser.find("title")