import sqlite3
from dataclasses import dataclass

@dataclass
class DBEntity:
    pass


@dataclass
class RankEntity(DBEntity):
    def __init__(self, word, url, title, score):
        self.word = word
        self.url = url
        self.title = title
        self.score = score


class SadhanaDB:
    def __init__(self, db=None):
        if db:
            self.db_name = db
        else:
            self.db_name = "def-sadhana-db"

        self.handle = sqlite3.connect(self.db_name)
        # self.handle.execute(
            # f'''
            # CREATE DATABASE \'{self.db_name}\'
            # '''
        # )

        self.handle.execute(
            '''
            create table if not exists `tbl_rank`(
            `id` integer primary key autoincrement,
            `word` varchar(128) not null,
            `url` varchar(512) not null,
            `title` varchar(1024) not null,
            `score` float not null default 0
            )
            '''
        ) 


    def close(self):
        self.handle.close()

    
    def insert_new_rank_item(self, entity: RankEntity):
        self.handle.execute(
            f'''
            insert into `tbl_rank`(word, url, title, score) 
            values(\'{entity.url}\', \'{entity.title}\', `{entity.score}`)
            '''
        )


    def get_rank_item(self, word: str):
        return self.handle.execute(
            f'''
            select * from `tbl_rank` where `word` = \'{word}\'
            '''
        )