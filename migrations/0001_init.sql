CREATE TYPE AuthProvider AS ENUM ('X');
CREATE TYPE BotType AS ENUM ('ALTITUDE', 'GROUNDSPEED');


CREATE TABLE Sessions (
    provider AuthProvider NOT NULL,
    bot_type BotType NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL
);


CREATE TABLE Flights (
    ident VARCHAR(255) PRIMARY KEY,
    ranking BotType NOT NULL,
    altitude INT,
    groundspeed INT,
    destination VARCHAR(255),
    origin VARCHAR(255)
);

