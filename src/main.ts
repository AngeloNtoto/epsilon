import { invoke } from "@tauri-apps/api/core";

window.addEventListener("DOMContentLoaded", () => {
  const bouton = document.querySelector("#pieger");

  bouton?.addEventListener("click", async () => {
    const message = await invoke<string>("pieger_dossiers");

    const resultat = document.querySelector("#resultat");

    if (resultat) {
      resultat.textContent = message;
    }
  });
});