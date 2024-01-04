#[rustfmt::skip]pub(crate) const GROUPS: wtx::database::schema_manager::EmbeddedMigrationsTy = &[{const INITIAL: &wtx::database::schema_manager::MigrationGroup<&'static str> = &wtx::database::schema_manager::MigrationGroup::new("INITIAL",1);const INITIAL_MIGRATIONS: &[wtx::database::schema_manager::UserMigrationRef<'static, 'static>] = &[wtx::database::schema_manager::UserMigrationRef::from_all_parts(7573493478190316387,&[],"create_author",None,"DROP TABLE author;","CREATE TABLE author (
  id INT NOT NULL PRIMARY KEY,
  first_name VARCHAR(50) NOT NULL,
  last_name VARCHAR(50) NOT NULL,
  email VARCHAR(100) NOT NULL
);",1),wtx::database::schema_manager::UserMigrationRef::from_all_parts(14432364634995446648,&[],"create_post",None,"DROP TABLE post;","CREATE TABLE post (
  id INT NOT NULL PRIMARY KEY,
  author_id INT NOT NULL,
  title VARCHAR(255) NOT NULL,
  description VARCHAR(500) NOT NULL,
  content TEXT NOT NULL
);",2),wtx::database::schema_manager::UserMigrationRef::from_all_parts(17129955996605416467,&[],"insert_author",None,"DELETE FROM author;","INSERT INTO author(id, first_name, last_name, email) VALUES
('1','Werner','Turcotte','bryce.wiza@example.com'),
('2','Issac','Schroeder','luisa01@example.com'),
('3','Lorenzo','Grant','granville.crist@example.com'),
('4','Leopoldo','Cartwright','kelsi79@example.org'),
('5','Melyssa','Wilkinson','modesto17@example.org'),
('6','Mack','Lubowitz','cayla14@example.net'),
('7','Gladys','Corkery','ebreitenberg@example.com'),
('8','Lizeth','Carroll','leopold52@example.com'),
('9','Morris','Becker','bhudson@example.com'),
('10','Samara','Tillman','stokes.leta@example.org'),
('11','Garfield','Friesen','ruthe36@example.org'),
('12','Brain','Dietrich','rickey.huel@example.com'),
('13','Nakia','Kreiger','madelynn.watsica@example.org'),
('14','Markus','Abernathy','kohler.steve@example.com'),
('15','Duncan','Hane','leffler.wendell@example.net'),
('16','Yasmeen','Bahringer','qcrona@example.com'),
('17','Grover','Cartwright','rcronin@example.net'),
('18','Sarah','Stokes','gavin.reinger@example.org'),
('19','Norwood','Hessel','torp.serena@example.org'),
('20','Carol','Fay','pacocha.ora@example.org');",3),wtx::database::schema_manager::UserMigrationRef::from_all_parts(13816380377167203395,&[],"insert_post",None,"DELETE FROM post;","INSERT INTO post(id, author_id, title, description, content) VALUES
('1','1','Laborum voluptatum est et.','Optio fugiat eveniet nihil voluptatem ea. Ut enim qui in ratione. Veritatis non in nisi est quis accusamus animi. Velit quasi ducimus nostrum deleniti consequatur sapiente nulla. Placeat dolores tempore totam amet occaecati minus error.','Aliquid ea quis reiciendis debitis omnis iste at. Voluptas quasi perspiciatis id officiis iste eligendi. Et quia et voluptatem ea. Aut incidunt alias quibusdam quas temporibus molestias facere porro.'),
('2','2','Ea aliquid suscipit mollitia ex adipisci.','Dolores explicabo sed illum nihil magnam animi quae temporibus. Omnis a nesciunt optio consequatur non. Veritatis omnis reprehenderit dolorem reprehenderit aspernatur. Natus id est nihil est voluptas aut omnis. Velit labore architecto explicabo labore.','Enim sit non eum veritatis. Aspernatur ipsa dolores quae perferendis. Nobis nemo nobis ab esse quia soluta. Magnam aperiam harum veritatis quos et laborum repudiandae.'),
('3','3','Dicta vero fugiat suscipit ut.','Quis quia qui modi voluptatem. Quo omnis consequatur et blanditiis quia consequuntur rerum. Ea dolorum magnam ab quibusdam. Velit fugit ratione ex qui quisquam aspernatur repellat.','Recusandae officia placeat enim ut aut animi. Atque minima hic quia repudiandae nobis et sed. Incidunt explicabo debitis eligendi sed. Reiciendis maiores velit atque ea ut.');",4),];(INITIAL,INITIAL_MIGRATIONS)},{const MORE_STUFF: &wtx::database::schema_manager::MigrationGroup<&'static str> = &wtx::database::schema_manager::MigrationGroup::new("MORE_STUFF",2);const MORE_STUFF_MIGRATIONS: &[wtx::database::schema_manager::UserMigrationRef<'static, 'static>] = &[wtx::database::schema_manager::UserMigrationRef::from_all_parts(8208328219135761847,&[],"create_stuff",None,"DROP TABLE coffee;
DROP TABLE apple;","CREATE TABLE apple (
  id INT NOT NULL PRIMARY KEY,
  weight INT NOT NULL
);

CREATE TABLE coffee (
  id INT NOT NULL PRIMARY KEY
);",1),wtx::database::schema_manager::UserMigrationRef::from_all_parts(7192396384181136034,&[],"insert_stuff",None,"","",2),wtx::database::schema_manager::UserMigrationRef::from_all_parts(16226037772308796192,&[wtx::database::DatabaseTy::Postgres,],"ultra_fancy_stuff",Some(wtx::database::schema_manager::Repeatability::Always),"","",3),];(MORE_STUFF,MORE_STUFF_MIGRATIONS)},];
