/// <reference path="./.sst/platform/config.d.ts" />

export default $config({
  app(input) {
    return {
      name: "bevy-game-api",
      removal: input?.stage === "production" ? "retain" : "remove",
      protect: ["production"].includes(input?.stage),
      home: "aws",
    };
  },
  async run() {

    const frontend = new sst.aws.StaticSite("BevyGameFrontend", {
      build: {
        command: "cd frontend && npm i && npm run build",
        output: "frontend/dist",
      },
    })

    const table = new sst.aws.Dynamo("BevyTable", {
      fields: {
        pk: "string",
        sk: "string",
        gsi1pk: "string",
        gsi1sk: "string",
        gsi2pk: "string",
        gsi2sk: "string",
      },
      primaryIndex: {
        hashKey: "pk",
        rangeKey: "sk",
      },
      globalIndexes: {
        "gsi1pk-gsi1sk-index": {
          hashKey: "gsi1pk",
          rangeKey: "gsi1sk",
        },
        "gsi2pk-gsi2sk-index": {
          hashKey: "gsi2pk",
          rangeKey: "gsi2sk",
        },
      },
    });

    const socket = new sst.aws.ApiGatewayWebSocket("SocketDemo", {
      transform: {
        route: {
          handler: {
            link: [table],
          },
        },
      },
    });

    const standaloneFunction = new sst.aws.Function("SocketBroadcaster", {
      handler: "socket/index.sendMessage",
      link: [socket, table],
    });

    socket.route("$connect", "socket/index.connect");
    socket.route("$disconnect", "socket/index.disconnect");
    // Explicitly handle chat messages on the sendMessage route
    socket.route("sendMessage", standaloneFunction.arn);

    return {
      socketUrl: socket.url,
      table: table.name,
      frontend: frontend.url
    }

  },
});
