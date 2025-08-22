import { list_files } from "ksj2gp";

console.log("Worker is loaded");

onmessage = async (event) => {
  const { file } = event.data;
  list_files(file);
};
