import "./styles.css";
import { invoke } from "@tauri-apps/api/core";

window.addEventListener("DOMContentLoaded", () => {
  const bouton = document.querySelector<HTMLButtonElement>("#pieger");
  const boutonRetirer = document.querySelector<HTMLButtonElement>("#retirer-points");
  const resultat = document.querySelector<HTMLParagraphElement>("#resultat");
  const statusPanel = document.querySelector<HTMLDivElement>("#status");
  const detailsPanel = document.querySelector<HTMLDivElement>("#details-panel");
  const detailsList = document.querySelector<HTMLUListElement>("#details-list");

  const setStatus = (message: string, level: "info" | "success" | "error" = "info") => {
    if (!resultat || !statusPanel) {
      return;
    }

    resultat.textContent = message;
    statusPanel.classList.remove("info", "success", "error");
    statusPanel.classList.add(level);
  };

  const setBusy = (busy: boolean) => {
    bouton?.toggleAttribute("disabled", busy);
    boutonRetirer?.toggleAttribute("disabled", busy);
  };

  const clearDetails = () => {
    if (detailsList) {
      detailsList.innerHTML = "";
    }
    if (detailsPanel) {
      detailsPanel.classList.remove("visible");
    }
  };

  const appendDetail = (text: string) => {
    if (!detailsList) {
      return;
    }

    const item = document.createElement("li");
    item.textContent = text;
    detailsList.appendChild(item);

    if (detailsPanel) {
      detailsPanel.classList.add("visible");
    }
  };

  const runAction = async (command: string, pendingMessage: string) => {
    clearDetails();
    if (command === "demasquer_dossiers") {
      appendDetail("🙋‍♂️ pardon ange");
    }
    setBusy(true);
    setStatus(pendingMessage, "info");

    try {
      const message = await invoke<string>(command);
      setStatus(message, "success");
      if (command === "demasquer_dossiers") {
        appendDetail("🙏 merci ya ange");
      }
    } catch (error) {
      console.error(`Erreur invoke ${command}:`, error);
      setStatus(`Erreur : ${String(error)}`, "error");
      appendDetail(`Erreur : ${String(error)}`);
    } finally {
      setBusy(false);
    }
  };

  bouton?.addEventListener("click", () => {
    void runAction("pieger_dossiers", "Analyse en cours…");
  });

  boutonRetirer?.addEventListener("click", () => {
    void runAction("demasquer_dossiers", "Suppression des points en cours…");
  });
});