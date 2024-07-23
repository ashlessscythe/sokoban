const { PrismaClient } = require("@prisma/client");
const { createObjectCsvWriter } = require("csv-writer");
const path = require("path");

const prisma = new PrismaClient();

async function exportTableToCsv(tableName) {
  let data;
  switch (tableName) {
    case "users":
      data = await prisma.users.findMany();
      break;
    case "punches":
      data = await prisma.punches.findMany();
      break;
    case "departments":
      data = await prisma.departments.findMany();
      break;
    case "admin_users":
      data = await prisma.admin_users.findMany();
      break;
    // Add more cases for other tables as needed
    default:
      console.error(`Table ${tableName} is not supported`);
      await prisma.$disconnect();
      return;
  }

  if (data.length === 0) {
    console.log(`No data found in table ${tableName}`);
    await prisma.$disconnect();
    return;
  }

  const csvWriter = createObjectCsvWriter({
    path: path.join(__dirname, `${tableName}.csv`),
    header: Object.keys(data[0]).map((key) => ({ id: key, title: key })),
  });

  await csvWriter.writeRecords(data);
  console.log(
    `Data from table ${tableName} has been exported to ${tableName}.csv`
  );

  await prisma.$disconnect();
}

const args = process.argv.slice(2);
if (args.length !== 1) {
  console.error("Usage: node exportToCsv.js <table_name>");
  process.exit(1);
}

const tableName = args[0];
exportTableToCsv(tableName);
