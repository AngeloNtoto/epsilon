import "./styles.css";
import { invoke } from "@tauri-apps/api/core";

window.addEventListener("DOMContentLoaded", () => {
  const bouton = document.querySelector<HTMLButtonElement>("#pieger");
  const resultat = document.querySelector<HTMLParagraphElement>("#resultat");
  const statusPanel = document.querySelector<HTMLDivElement>("#status");

  const setStatus = (message: string, level: "info" | "success" | "error" = "info") => {
    if (!resultat || !statusPanel) {
      return;
    }

    resultat.textContent = message;
    statusPanel.classList.remove("info", "success", "error");
    statusPanel.classList.add(level);
  };

  bouton?.addEventListener("click", async () => {
    if (!bouton) {
      return;
    }

    bouton.disabled = true;
    setStatus("Analyse en cours…", "info");

    try {
      const message = await invoke<string>("pieger_dossiers");
      setStatus(message, "success");
    } catch (error) {
      console.error("Erreur invoke pieger_dossiers:", error);
      setStatus(`Erreur : ${String(error)}`, "error");
    } finally {
      bouton.disabled = false;
    }
  });
});