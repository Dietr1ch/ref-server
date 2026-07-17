CREATE FUNCTION prevent_immutable_user_fields() RETURNS trigger AS $$
BEGIN
  IF NEW.id IS DISTINCT FROM OLD.id THEN
    RAISE EXCEPTION 'column "id" is immutable';
  END IF;
  IF NEW.created_at IS DISTINCT FROM OLD.created_at THEN
    RAISE EXCEPTION 'column "created_at" is immutable';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_immutable_fields
  BEFORE UPDATE ON users
  FOR EACH ROW
  EXECUTE FUNCTION prevent_immutable_user_fields();
