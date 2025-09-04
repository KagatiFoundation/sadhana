import mysql.connector as mcon
from dataclasses import dataclass
from collections import Counter

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
            database="sadhana_main_db",
            user="root",
            password="",
            host="localhost"
        )

        self.buffered_cursor = self.handle.cursor(buffered=True)


    def close(self):
        self.handle.close()


    def batch_insert_new_rank_items(self, terms, url, title):
        """
        Inserts a new URL and its terms into the database.
        Updates document frequency (DF) in tbl_word and inserts term frequency (TF) into tbl_word_doc.
        """

        # Use a single cursor and transaction
        cur = self.handle.cursor()
        try:
            self.handle.start_transaction()

            url_entry_query = '''
            INSERT INTO tbl_doc_urls(url, title)
            VALUES (%s, %s)
            ON DUPLICATE KEY UPDATE url_id = LAST_INSERT_ID(url_id);
            '''
            cur.execute(url_entry_query, (url, title))
            url_id = cur.lastrowid

            word_counts = Counter(terms)

            freq_query = '''
            INSERT INTO tbl_words(word, freq)
            VALUES (%s, 1)
            ON DUPLICATE KEY UPDATE freq = freq + 1;
            '''
            cur.executemany(freq_query, [(word,) for word in word_counts.keys()])

            placeholders = ','.join(['%s'] * len(word_counts))
            cur.execute(f"SELECT id, word FROM tbl_words WHERE word IN ({placeholders})", list(word_counts.keys()))
            word_ids = {row[1]: row[0] for row in cur.fetchall()}

            tf_data = [(word_ids[word], url_id, count) for word, count in word_counts.items()]
            insert_tf_query = '''
            INSERT INTO tbl_word_doc(word_id, url_id, word_freq)
            VALUES (%s, %s, %s)
            '''
            cur.executemany(insert_tf_query, tf_data)

            self.handle.commit()
        except mcon.Error as e:
            self.handle.rollback()
            print("Database error:", e)
        finally:
            cur.close()

    
    def insert_new_word_doc_mapping(self, entity: WordDocMapping):
        self.handle.cursor().execute(
            f'''
            insert into `tbl_word_doc_scoring`(word, doc_id, doc_title, score) 
            values(\'{entity.word}\', \'{entity.url}\', \'{entity.title}\', `{entity.score}`);
            '''
        )