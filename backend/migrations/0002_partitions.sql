DO $$
DECLARE
    base_month DATE := date_trunc('month', NOW())::date;
    start_date DATE;
    end_date DATE;
    idx INT;
BEGIN
    FOR idx IN 0..1 LOOP
        start_date := (base_month + (idx || ' month')::interval)::date;
        end_date := (start_date + interval '1 month')::date;

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS dht_observations_%s PARTITION OF dht_observations FOR VALUES FROM (%L) TO (%L)',
            to_char(start_date, 'YYYYMM'),
            start_date,
            end_date
        );

        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS search_audit_logs_%s PARTITION OF search_audit_logs FOR VALUES FROM (%L) TO (%L)',
            to_char(start_date, 'YYYYMM'),
            start_date,
            end_date
        );
    END LOOP;
END;
$$;

CREATE TABLE IF NOT EXISTS dht_observations_default
PARTITION OF dht_observations DEFAULT;

CREATE TABLE IF NOT EXISTS search_audit_logs_default
PARTITION OF search_audit_logs DEFAULT;
