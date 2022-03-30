-- ----------------------------
-- Function structure for trigger_set_timestamp
-- ----------------------------
CREATE OR REPLACE FUNCTION "trigger_set_timestamp"()
  RETURNS "trigger" AS $BODY$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$BODY$
  LANGUAGE plpgsql VOLATILE
  COST 100;

-- ----------------------------
-- Table structure for accounts
-- ----------------------------
CREATE TABLE "accounts" (
  "id" varchar NOT NULL,
  "email" varchar NOT NULL,
  "password" varchar NOT NULL,
  "name" varchar NOT NULL,
  "is_superuser" bool NOT NULL DEFAULT false,
  "last_login_at" timestamptz(6),
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6),
  CONSTRAINT "accounts_pkey" PRIMARY KEY ("id")
);
CREATE UNIQUE INDEX "uniq_email" ON "accounts" USING btree ("email" "text_ops" ASC NULLS LAST);
CREATE TRIGGER "on_accounts_update" BEFORE UPDATE ON "accounts" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();

-- ----------------------------
-- Table structure for schemas
-- ----------------------------
CREATE TABLE "schemas" (
  "id" varchar NOT NULL,
  "name" varchar NOT NULL,
  "account_id" varchar NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6),
  CONSTRAINT "schemas_pkey" PRIMARY KEY ("id"),
  CONSTRAINT "fk_account_id" FOREIGN KEY ("account_id") REFERENCES "accounts" ("id") ON DELETE CASCADE ON UPDATE NO ACTION
);
CREATE TRIGGER "on_schemas_update" BEFORE UPDATE ON "schemas" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();

-- ----------------------------
-- Table structure for fields
-- ----------------------------
CREATE TYPE data_type AS ENUM ('string', 'number', 'integer', 'boolean', 'time');
CREATE TABLE "fields" (
  "id" varchar NOT NULL,
  "schema_id" varchar NOT NULL,
  "identifier" varchar NOT NULL,
  "data_type" data_type NOT NULL,
  "comment" varchar,
  "unit" varchar,
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6),
  CONSTRAINT "fields_pkey" PRIMARY KEY ("id"),
  CONSTRAINT "fk_schema_id" FOREIGN KEY ("schema_id") REFERENCES "schemas" ("id") ON DELETE CASCADE ON UPDATE NO ACTION
);
CREATE TRIGGER "on_fields_update" BEFORE UPDATE ON "fields" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();

-- ----------------------------
-- Table structure for devices
-- ----------------------------
CREATE TABLE "devices" (
  "id" varchar NOT NULL,
  "account_id" varchar NOT NULL,
  "schema_id" varchar NOT NULL,
  "name" varchar NOT NULL,
  "label_version" int8 NOT NULL,
  "is_active" bool NOT NULL DEFAULT true,
  "is_online" bool NOT NULL DEFAULT false,
  "mqtt_username" varchar NOT NULL,
  "mqtt_password" varchar NOT NULL,
  "is_super_device" bool NOT NULL DEFAULT false,
  "acl_pubs" jsonb NOT NULL DEFAULT '[]',
  "acl_subs" jsonb NOT NULL DEFAULT '[]',
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6),
  CONSTRAINT "devices_pkey" PRIMARY KEY ("id"),
  CONSTRAINT "fk_account_id" FOREIGN KEY ("account_id") REFERENCES "accounts" ("id") ON DELETE CASCADE ON UPDATE NO ACTION,
  CONSTRAINT "fk_schema_id" FOREIGN KEY ("schema_id") REFERENCES "schemas" ("id") ON DELETE CASCADE ON UPDATE NO ACTION
);
CREATE INDEX "idx_acl_pubs" ON "devices" USING gin ("acl_pubs");
CREATE INDEX "idx_acl_subs" ON "devices" USING gin ("acl_subs");
CREATE TRIGGER "on_devices_update" BEFORE UPDATE ON "devices" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();

-- ----------------------------
-- Table structure for labels
-- ----------------------------
CREATE TABLE "labels" (
  "id" varchar NOT NULL,
  "account_id" varchar NOT NULL,
  "name" varchar NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6),
  CONSTRAINT "labels_pkey" PRIMARY KEY ("id"),
  CONSTRAINT "fk_account_id" FOREIGN KEY ("account_id") REFERENCES "accounts" ("id") ON DELETE CASCADE ON UPDATE NO ACTION
);
CREATE TRIGGER "on_labels_update" BEFORE UPDATE ON "labels" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();

-- ----------------------------
-- Table structure for labels_device_relation
-- ----------------------------
CREATE TABLE "labels_device_relation" (
  "id" SERIAL8 NOT NULL,
  "label_id" varchar NOT NULL,
  "device_id" varchar NOT NULL,
  CONSTRAINT "labels_device_relation_pkey" PRIMARY KEY ("id"),
  CONSTRAINT "fk_label_id" FOREIGN KEY ("label_id") REFERENCES "labels" ("id") ON DELETE CASCADE ON UPDATE NO ACTION,
  CONSTRAINT "fk_device_id" FOREIGN KEY ("device_id") REFERENCES "devices" ("id") ON DELETE CASCADE ON UPDATE NO ACTION
);
CREATE TRIGGER "on_labels_device_relation_update" BEFORE UPDATE ON "labels_device_relation" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();

-- ----------------------------
-- Table structure for command_request_logs
-- ----------------------------
CREATE TABLE "command_request_logs" (
  "message_id" varchar NOT NULL,
  "topic" varchar NOT NULL,
  "command" varchar NOT NULL,
  "mode" varchar NOT NULL,
  "body" varchar NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "device_id" varchar NOT NULL,
  CONSTRAINT "command_request_logs_pkey" PRIMARY KEY ("message_id"),
  CONSTRAINT "fk_device_id" FOREIGN KEY ("device_id") REFERENCES "devices" ("id") ON DELETE CASCADE ON UPDATE NO ACTION
);

-- ----------------------------
-- Table structure for command_response_logs
-- ----------------------------
CREATE TABLE "command_response_logs" (
  "id" varchar NOT NULL,
  "message_id" varchar NOT NULL,
  "payload" varchar NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  CONSTRAINT "command_response_logs_pkey" PRIMARY KEY ("id"),
  CONSTRAINT "fk_message_id" FOREIGN KEY ("message_id") REFERENCES "command_request_logs" ("message_id") ON DELETE CASCADE ON UPDATE NO ACTION
);

-- ----------------------------
-- Table structure for device_connections
-- ----------------------------
CREATE TABLE "device_connections" (
  "id" varchar NOT NULL,
  "connected" bool NOT NULL,
  "client_id" varchar NOT NULL,
  "node" varchar NOT NULL,
  "keep_alive" varchar NOT NULL,
  "ip_address" varchar NOT NULL,
  "proto_ver" int8 NOT NULL,
  "connected_at" timestamptz(6) NOT NULL,
  "disconnected_at" timestamptz(6) NOT NULL,
  "disconnected_reason" varchar NOT NULL,
  "created_at" timestamptz(6) NOT NULL DEFAULT now(),
  "updated_at" timestamptz(6),
  "device_id" varchar NOT NULL,
  CONSTRAINT "device_connections_pkey" PRIMARY KEY ("id"),
  CONSTRAINT "fk_device_id" FOREIGN KEY ("device_id") REFERENCES "devices" ("id") ON DELETE CASCADE ON UPDATE NO ACTION
);
CREATE UNIQUE INDEX "uniq_device_client" ON "device_connections" USING btree (
  "device_id" "text_ops" ASC NULLS LAST,
  "client_id" "text_ops" ASC NULLS LAST
);
CREATE TRIGGER "on_device_connections_update" BEFORE UPDATE ON "device_connections" FOR EACH ROW EXECUTE PROCEDURE "trigger_set_timestamp"();
