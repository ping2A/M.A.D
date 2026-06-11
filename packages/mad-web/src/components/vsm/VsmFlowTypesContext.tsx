import { createContext, useContext } from "react";
import type { VsmFlowTypeDef } from "../../types";
import { DEFAULT_FLOW_TYPES } from "../../utils/valueStream";

const VsmFlowTypesContext = createContext<VsmFlowTypeDef[]>(DEFAULT_FLOW_TYPES);

export function VsmFlowTypesProvider({
  flowTypes,
  children,
}: {
  flowTypes: VsmFlowTypeDef[];
  children: React.ReactNode;
}) {
  return (
    <VsmFlowTypesContext.Provider value={flowTypes}>{children}</VsmFlowTypesContext.Provider>
  );
}

export function useVsmFlowTypes(): VsmFlowTypeDef[] {
  return useContext(VsmFlowTypesContext);
}
