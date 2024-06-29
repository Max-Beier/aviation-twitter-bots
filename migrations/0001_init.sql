CREATE TYPE AuthProvider AS ENUM ('X');

CREATE TABLE Sessions (
    provider AuthProvider NOT NULL,
    access_token TEXT NOT NULL
);

CREATE TABLE Flights (
    ident VARCHAR(255) PRIMARY KEY,
    altitude INT,
    groundspeed INT,
    destination VARCHAR(255),
    origin VARCHAR(255)
);

