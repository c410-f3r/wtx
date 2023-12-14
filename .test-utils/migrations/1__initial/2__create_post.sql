-- wtx IN

CREATE TABLE post (
  id INT NOT NULL PRIMARY KEY,
  author_id INT NOT NULL,
  title VARCHAR(255) NOT NULL,
  description VARCHAR(500) NOT NULL,
  content TEXT NOT NULL
);

-- wtx OUT

DROP TABLE post;
