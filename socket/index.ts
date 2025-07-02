import {
  APIGatewayProxyEvent,
  APIGatewayProxyResult,
  Context,
} from "aws-lambda";
import {
  ApiGatewayManagementApiClient,
  PostToConnectionCommand,
} from "@aws-sdk/client-apigatewaymanagementapi";
import { ConnectionEntity } from "./models/connection";

export const connect = async (
  event: APIGatewayProxyEvent,
  context: Context
): Promise<APIGatewayProxyResult> => {
  const connection = event.requestContext.connectionId;
  console.log("🟢 Connection Initiated", connection);

  try {
    const result = await ConnectionEntity.put({
      connection: connection,
      user: "anonymous",
    }).go();
    console.log("🟢 Connection saved", result);

    return { statusCode: 200, body: "Connected." };
  } catch (error) {
    console.error("Error connecting", error);
    return { statusCode: 500, body: "Failed to connect." };
  }
};

export const disconnect = async (
  event: APIGatewayProxyEvent,
  context: Context
): Promise<APIGatewayProxyResult> => {
  const connection = event.requestContext.connectionId;
  console.log("🔴 Connection Disconnected", connection);

  try {
    await ConnectionEntity.delete({ connection }).go();
    return { statusCode: 200, body: "Disconnected." };
  } catch (error) {
    console.error("Error disconnecting", error);
    return { statusCode: 500, body: "Failed to disconnect." };
  }
};

export const defaultHandler = async (
  event: APIGatewayProxyEvent,
  context: Context
): Promise<APIGatewayProxyResult> => {
  console.log("🟡 Default handler", event);
  return {
    statusCode: 200,
    body: JSON.stringify({ message: "Hello, world!" }),
  };
};

export const sendMessage = async (
  event: APIGatewayProxyEvent,
  context: Context
): Promise<APIGatewayProxyResult> => {
  console.log("🟣 Sending message to all connections", event.body);
  console.log("Request context", event.requestContext);
  console.log("Context", context);
  console.log(
    `ConnectionId ${event.requestContext.connectionId} domain ${event.requestContext.domainName} stage ${event.requestContext.stage}`
  );

  const apiGatewayManagementApi = new ApiGatewayManagementApiClient({
    endpoint: `https://${event.requestContext.domainName}/${event.requestContext.stage}`,
  });
  console.log("Management API endpoint", apiGatewayManagementApi.config.endpoint);

  try {
    const result = await ConnectionEntity.find({}).go();

    if (!result.data || result.data.length === 0) {
      console.warn("⚠️ No active connections found.");
      return { statusCode: 200, body: "No connections to send to." };
    }

    console.log(`🟢 Found ${result.data.length} connections`);

    const postCalls = result.data.map(async (item) => {
      const connection = item.connection;
      console.log(`🔹 Sending to connection: ${connection}`);

      try {
        await apiGatewayManagementApi.send(
          new PostToConnectionCommand({
            ConnectionId: connection,
            Data: Buffer.from(event.body ?? "{}"),
          })
        );
      } catch (error) {
        if (error.statusCode === 410) {
          console.log(`🚨 Stale connection found, deleting: ${connection}`);
          await ConnectionEntity.delete({ connection }).go();
        } else {
          console.error(`❌ Error sending message to ${connection}`, error);
        }
      }
    });

    await Promise.all(postCalls);

    return { statusCode: 200, body: "Message sent to all connections." };
  } catch (error) {
    console.error("🔥 Error sending messages", error);
    return { statusCode: 500, body: "Failed to send messages." };
  }
};
