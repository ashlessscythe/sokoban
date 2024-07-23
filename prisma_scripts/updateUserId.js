const { PrismaClient } = require("@prisma/client");
const fs = require("fs");
const path = require("path");
const csv = require("csv-parser");

const prisma = new PrismaClient();

async function updateUserId(oldId, newId) {
  try {
    await prisma.$transaction(async (prisma) => {
      await prisma.users.update({
        where: { user_id: oldId },
        data: { user_id: newId },
      });
    });
    console.log(`User ID updated from ${oldId} to ${newId}`);
  } catch (error) {
    if (error.code === "P2025") {
      console.error(
        `Error: Record with user_id ${oldId} not found. Cannot update to ${newId}.`
      );
    } else {
      console.error(`Error updating user ID from ${oldId} to ${newId}:`, error);
    }
  }
}

async function processCSV(filePath) {
  const updates = [];

  fs.createReadStream(filePath)
    .pipe(csv())
    .on("data", (row) => {
      updates.push({ oldId: row.old_id, newId: row.new_id });
    })
    .on("end", async () => {
      console.log("CSV file successfully processed");
      for (const update of updates) {
        await updateUserId(update.oldId, update.newId);
      }
      await prisma.$disconnect();
    })
    .on("error", (error) => {
      console.error("Error reading CSV file:", error);
    });
}

// get csv path from arg
const args = process.argv.slice(2);
if (args.length !== 1) {
  console.error("Usage: node updateUserId.js <csv_file_path>");
  process.exit(1);
}

const csvFilePath = path.resolve(args[0]);
processCSV(csvFilePath);
