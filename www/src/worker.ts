import { read_bytes } from "ksj2gp";

console.log("Worker is loaded");

onmessage = async (event) => {
  const { file } = event.data;
  read_bytes(file, 1, 2);
};
