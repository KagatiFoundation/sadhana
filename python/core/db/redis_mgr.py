import redis

class SadhanaRedisMgr:
    def __init__(self):
        self.handle = redis.Redis()


    def get_internal_value(self, key: str):
        self.handle.get(f'internal-{key}')


    def incr_internal_value(self, key: str):
        self.handle.incr(f'internal-{key}', 1)


    def set_internal_value(self, key: str, value: str):
        self.handle.set(f'internal-{key}', value)