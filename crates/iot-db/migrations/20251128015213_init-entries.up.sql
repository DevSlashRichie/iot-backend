CREATE TABLE sensor (
    id BINARY(16) NOT NULL PRIMARY KEY,
    label VARCHAR(255) NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE sensor_entries (
    id BINARY(16) NOT NULL PRIMARY KEY,
    sensor_id BINARY(16) NOT NULL,
    value FLOAT NOT NULL,
    created_at BIGINT NOT NULL,

    CONSTRAINT fk_sensor
        FOREIGN KEY (sensor_id)
        REFERENCES sensor(id)
        ON DELETE CASCADE
);


