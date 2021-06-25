import { useState } from "react";

const apiKeyKey = "api_key";

export const getApiKey = () => localStorage.getItem(apiKeyKey);

export default function useApiKey() {
  const storedApiKey = getApiKey();
  const [apiKey, setApiKey] = useState(storedApiKey);

  const saveApiKey = (userApiKey: string | null) => {
    if (userApiKey) {
      localStorage.setItem(apiKeyKey, userApiKey);
      setApiKey(userApiKey);
    }
  };

  return {
    setApiKey: saveApiKey,
    apiKey,
  };
}
