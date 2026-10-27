import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface GermanPhoneticCorrectionProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

// Tonfeder: matches custom words by German sound (Cologne phonetics).
export const GermanPhoneticCorrection: React.FC<GermanPhoneticCorrectionProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const enabled = getSetting("german_phonetic_correction") ?? true;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(nextEnabled) =>
          updateSetting("german_phonetic_correction", nextEnabled)
        }
        isUpdating={isUpdating("german_phonetic_correction")}
        label={t("settings.advanced.germanPhonetic.title")}
        description={t("settings.advanced.germanPhonetic.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  });
