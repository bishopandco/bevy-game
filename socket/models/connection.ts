import { Entity } from "electrodb";
import { DynamoDBClient } from "@aws-sdk/client-dynamodb";

const client = new DynamoDBClient({ region: "us-east-1" });
const table = "bevy-game-api-production-BevyTableTable-hwsbrscz";

export interface ConnectionsCollection {
  connections: Connection[];
}

class Connection {
  connection: string;
  user: string;
  connectedAt: string;

  constructor(connection: string, user: string) {
    this.connection = connection;
    this.user = user;
    this.connectedAt = new Date().toISOString();
  }

  all(): Connection[] {
    return [];
  }

  create(connection: Connection): void {}

  get(connection: string): Connection | undefined {
    return undefined;
  }

  update(connection: string, updatedConnection: Partial<Connection>): boolean {
    return false;
  }

  destroy(connection: string): boolean {
    const connectionResult = this.get(connection);
    if (!connectionResult) {
      return false;
    } else {
      ConnectionEntity.delete({
        connection: connectionResult.connection,
        connectedAt: connectionResult.connectedAt,
      }).go();
    }

    return false;
  }
}

export const ConnectionEntity = new Entity(
  {
    model: {
      entity: "connection",
      version: "1",
      service: "connections",
    },
    attributes: {
      connection: {
        type: "string",
        required: true,
      },
      user: {
        type: "string",
        required: true,
      },
      connectedAt: {
        type: "string",
        default: () => new Date().toISOString(),
      },
    },
    indexes: {
      connection: {
        pk: {
          field: "pk",
          composite: ["connection"],
        },
        sk: {
          field: "sk",
          composite: [],
        },
      },
      byUser: {
        index: "gsi1pk-gsi1sk-index",
        pk: {
          field: "gsi1pk",
          composite: ["user"],
        },
        sk: {
          field: "gsi1sk",
          composite: ["connectedAt"],
        },
      },
    },
  },
  {
    table,
    client,
  }
);

export async function getAllConnectionsPaginated(lastEvaluatedKey?: string) {
  // TODO Implement pagination
  try {
    console.info("Disregarding lastEvaluatedKey", lastEvaluatedKey);
    const result = await ConnectionEntity.find({}).go();

    return {
      connections: result.data,
    };
  } catch (error) {
    console.error("Error fetching connections", error);
    throw error;
  }
}
