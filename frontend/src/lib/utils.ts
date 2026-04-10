import type { Address } from "@solana/addresses";
import { toast } from "react-toastify";

export const copyAddressHelper = async (
  address: Address | undefined,
): Promise<boolean> => {
  if (!address) return false;
  try {
    await navigator.clipboard.writeText(address);
    toast.success(`Copied address to clipboard!`, {
      pauseOnFocusLoss: false,
      autoClose: 3000,
    });
    return true;
  } catch {
    toast.error(`Failed to copy address to clipboard`, {
      pauseOnFocusLoss: false,
      autoClose: 3000,
    });
    return false;
  }
};
