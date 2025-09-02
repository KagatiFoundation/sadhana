import redis

class SadhanaRedisMgr:
    def __init__(self):
        self.handle = redis.Redis()


    def get_term_freq(self, term: str) -> int:
        return self.handle.hget('FREQ', term)

    
    def store_term_freq(self, term: str, amount: int):
        return self.handle.hincrby('FREQ', term, amount)


    def get_internal_value(self, key: str):
        return self.handle.get(f'internal-{key}')


    def incr_internal_value(self, key: str):
        self.handle.incr(f'internal-{key}')


    def decr_internal_value(self, key: str):
        self.handle.decr(f'internal-{key}')


    def set_internal_value(self, key: str, value: str):
        self.handle.set(f'internal-{key}', value)