CREATE TABLE home_settings (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    introduction TEXT NOT NULL DEFAULT '' CHECK (char_length(introduction) <= 500),
    featured_project_id UUID REFERENCES project(id) ON DELETE SET NULL
);
INSERT INTO home_settings (singleton) VALUES (TRUE);
