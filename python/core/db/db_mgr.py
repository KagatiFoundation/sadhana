import mysql.connector as mcon
from dataclasses import dataclass

@dataclass
class DBEntity:
    pass


@dataclass
class WordDocMapping(DBEntity):
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

        self.handle = mcon.connect(
            database="sadhana_main_db"
        )


    def close(self):
        self.handle.close()

    
    def insert_new_word_doc_mapping(self, entity: WordDocMapping):
        self.handle.execute(
            f'''
            insert into `tbl_word_doc_scoring`(word, doc_id, doc_title, score) 
            values(\'{entity.word}\', \'{entity.url}\', \'{entity.title}\', `{entity.score}`);
            '''
        )


    def get_word_doc_mapping(self, word: str):
        return self.handle.execute(
            f'''
            select * from `tbl_word_doc_scoring` where `word` = \'{word}\';
            '''
        )