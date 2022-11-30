-- migrations/{timespamp}_create_subscription_table.sql
-- Create Subscription Table
CREATE TABLE subscriptions (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE, -- uniqueness enforced here. This can be a bottleneck for throughput
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL -- timestamptz is a time-zone aware date and time type
);


