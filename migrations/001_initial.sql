CREATE TYPE user_role AS ENUM ('student', 'recruiter');
CREATE TYPE position_status AS ENUM ('draft', 'open', 'closed');
CREATE TYPE application_status AS ENUM ('pending', 'reviewing', 'accepted', 'rejected');


CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name     TEXT NOT NULL,
    role          user_role NOT NULL,
    created_at    TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE positions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recruiter_id    UUID REFERENCES users(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL,
    location        TEXT NOT NULL,
    is_remote       BOOLEAN DEFAULT false,
    has_salary      BOOLEAN DEFAULT false,
    salary_amount   NUMERIC,
    salary_currency TEXT DEFAULT 'DOP',
    status          position_status DEFAULT 'draft',
    created_at      TIMESTAMPTZ DEFAULT now()
);


CREATE TABLE applications (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id    UUID REFERENCES users(id) ON DELETE CASCADE,
    position_id   UUID REFERENCES positions(id) ON DELETE CASCADE,
    cv_url        TEXT NOT NULL,
    cover_letter  TEXT,
    status        application_status DEFAULT 'pending',
    applied_at    TIMESTAMPTZ DEFAULT now(),
    UNIQUE(student_id, position_id)
);