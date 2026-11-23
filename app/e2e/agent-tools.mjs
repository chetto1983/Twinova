// The agent a package installs answers an MCP client (spec 2026-11-23-app-mcp-design.md, test plan, "The release"):
// CI runs it on the twinova-agent taken out of the deb and the MSI. Headless, it lists its tools and opens an example.
//
//   npm run agent-check -- <path to twinova-agent>

import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

const [agent] = process.argv.slice(2);
if (!agent) throw new Error("usage: node e2e/agent-tools.mjs <path to twinova-agent>");
const client = new Client({ name: "twinova-package-check", version: "1" });
await client.connect(new StdioClientTransport({ command: agent, args: ["--headless"], stderr: "inherit" }));
try {
  const { tools } = await client.listTools();
  const names = tools.map((t) => t.name);
  console.log(`${agent}: ${names.length} tools`);
  for (const needed of ["list_examples", "open_example", "step", "snapshot"]) {
    if (!names.includes(needed)) throw new Error(`no ${needed} among ${names.join(", ")}`);
  }
  const opened = await client.callTool({ name: "open_example", arguments: { id: "demo_m3" } });
  if (opened.isError) throw new Error(JSON.stringify(opened.content));
  console.log(`opened the example: ${opened.structuredContent.name}`);
} finally {
  await client.close();
}
