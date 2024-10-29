const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;
let statusE1;
let excel_file_status;
let cs_file_status;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

async function open_excel_file() {
  excel_file_status.textContent = await invoke("open_file");
}

async function open_cs_file() {
  cs_file_status.textContent = await invoke("open_file");
}

async function convert_word() {
  // const excelPath = excel_file_status.textContent;
  let excelPath = excel_file_status.textContent;
  let csPath = cs_file_status.textContent;

  // 백슬래시를 이중으로 바꿔줍니다.
  excelPath = excelPath.replace(/\\/g, "\\\\");
  csPath = csPath.replace(/\\/g, "\\\\");

  // if (excelPath && csPath) {
  //   statusE1.textContent = `Converting using excel: ${excelPath} and cs: ${csPath}`;
  //   statusE1.textContent = await invoke("convert_word", { excel_add: excelPath, cs_add: csPath });
  // } else {
  //   statusE1.textContent = "Please select both Excel and CS files before converting.";
  // }

  try {
    statusE1.textContent = await invoke("convert_word", { excel_add: excelPath, cs_add: csPath });
  } catch (error) {
    console.error("Error invoking convert_word:", error);
    statusE1.textContent = "Error occurred: " + error.message;
  }
}

async function renumbering_tc() {
  let csPath = cs_file_status.textContent;

  // 백슬래시를 이중으로 바꿔줍니다.
  csPath = csPath.replace(/\\/g, "\\\\");

  try {
    statusE1.textContent = await invoke("renumbering_tc", { cs_path: csPath });
  } catch (error) {
    console.error("Error invoking renumbering_tc:", error);
    statusE1.textContent = "Error occurred " + error.message;
  }
}

window.addEventListener("DOMContentLoaded", () => {

  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  excel_file_status = document.querySelector("#excel-status");
  cs_file_status = document.querySelector("#cs-status");
  statusE1 = document.querySelector("#status");

  document.querySelector("#excel-file").addEventListener("click", (e) => {
    open_excel_file();
  });

  document.querySelector("#cs-file").addEventListener("click", (e) => {
    open_cs_file();
  });

  document.querySelector("#convert-word").addEventListener("click", (e) => {
    convert_word();
  });

  document.querySelector("#sorting-TCNum").addEventListener("click", (e) => {
    renumbering_tc();
  });

});
