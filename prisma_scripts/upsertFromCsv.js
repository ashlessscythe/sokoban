const { PrismaClient } = require("@prisma/client");
const fs = require("fs");
const path = require("path");
const csv = require("csv-parser");

const prisma = new PrismaClient();

// currently only supports users table

async function upsertRecord(tableName, record) {
  switch (tableName) {
    case "users":
      await prisma.users.upsert({
        where: { user_id: record.user_id },
        update: {
          name: record.name || "Unknown", // Default name
          email: record.email || "unknown@example.com", // Default email
          dept_id: record.dept_id || 1, // Default department ID
          profile_picture: record.profile_picture || null, // Default profile picture
          created_at: record.created_at || new Date(), // Default created_at to now
        },
        create: {
          user_id: record.user_id,
          name: record.name || "Unknown", // Default name
          email: record.email || "unknown@example.com", // Default email
          dept_id: record.dept_id || 1, // Default department ID
          profile_picture: record.profile_picture || null, // Default profile picture
          created_at: record.created_at || new Date(), // Default created_at to now
        },
      });
      console.log(`User with ID ${record.user_id} upserted successfully.`);
      break;
    // Add more cases for other tables as needed
    default:
      console.error(`Table ${tableName} is not supported`);
      break;
  }
}

async function processCSV(tableName, filePath) {
  const records = [];

  fs.createReadStream(filePath)
    .pipe(csv())
    .on("data", (row) => {
      const record = {
        user_id: row.user_id,
        name: row.name,
        email: row.email,
        dept_id: row.dept_id ? parseInt(row.dept_id, 10) : undefined,
        profile_picture: row.profile_picture,
        created_at: row.created_at ? new Date(row.created_at) : undefined,
      };
      records.push(record);
    })
    .on("end", async () => {
      console.log("CSV file successfully processed");
      for (const record of records) {
        await upsertRecord(tableName, record);
      }
      await prisma.$disconnect();
    })
    .on("error", (error) => {
      console.error("Error reading CSV file:", error);
    });
}

const args = process.argv.slice(2);
if (args.length !== 1) {
  console.error("Usage: node upsertFromCsv.js <csv_file_path>");
  process.exit(1);
}

const csvFilePath = path.resolve(args[0]);
const tableName = path.basename(csvFilePath, path.extname(csvFilePath));
processCSV(tableName, csvFilePath);
