-- wtx IN

CREATE TABLE apple (
  id INT NOT NULL PRIMARY KEY,
  weight INT NOT NULL
);

CREATE TABLE coffee (
  id INT NOT NULL PRIMARY KEY
);

-- wtx OUT

DROP TABLE coffee;
DROP TABLE apple;