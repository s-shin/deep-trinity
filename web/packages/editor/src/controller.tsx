import React from "react";
import { Box } from "@chakra-ui/react";

export type ControllerProps = {
  children?: React.ReactNode;
};

export function Controller(props: ControllerProps) {
  return (
    <Box bg={"colors.gray.100"} borderRadius={"3px"} width>
    </Box>
  );
}

export default Controller;
