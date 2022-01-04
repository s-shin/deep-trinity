import React from "react";
import { Accordion, AccordionDetails, AccordionSummary, Typography } from "@mui/material";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";

export type PanelProps = {
  title: string,
};

export const Panel: React.FC<PanelProps> = props => {
  return (
    <Accordion>
      <AccordionSummary expandIcon={<ExpandMoreIcon/>}>
        <Typography>{props.title}</Typography>
      </AccordionSummary>
      <AccordionDetails>
        {props.children}
      </AccordionDetails>
    </Accordion>
  );
};