import React from "react";
import { Container, Table } from "semantic-ui-react";
import { Rule } from "../../Models/Rule";
import RuleComponent from "./Rule";

type RuleErrors = { [rule: number]: string | undefined };

interface Props {
  onSetRuleRequested: (rule: Rule) => void;
  onDeleteRuleRequested: (rule: Rule) => void;
  rules: Rule[];
  errors: RuleErrors;
}

const RulesTable: React.FC<Props> = ({ onSetRuleRequested, onDeleteRuleRequested, rules, errors }) => {
  const ruleSort = (a: Rule, b: Rule) => {
    if (a.priority < b.priority) {
      return -1;
    }
    if (a.priority > b.priority) {
      return 1;
    }
    if (a.ruleName < b.ruleName) {
      return -1;
    }
    if (a.ruleName > b.ruleName) {
      return 1;
    }
    return 0;
  };

  return (
    <Container fluid>
      <Table>
        <Table.Header>
          <Table.Row>
            <Table.HeaderCell>Priority</Table.HeaderCell>
            <Table.HeaderCell>Name</Table.HeaderCell>
            <Table.HeaderCell>Description template</Table.HeaderCell>
            <Table.HeaderCell>Match field</Table.HeaderCell>
            <Table.HeaderCell>Regex</Table.HeaderCell>
            <Table.HeaderCell>Postings</Table.HeaderCell>
            <Table.HeaderCell />
            <Table.HeaderCell />
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {rules.sort(ruleSort).map((r, index) => (
            <RuleComponent
              key={index}
              rule={r}
              error={errors[r.id]}
              onSet={onSetRuleRequested}
              onDelete={onDeleteRuleRequested}
            />
          ))}
        </Table.Body>
      </Table>
    </Container>
  );
};

export default RulesTable;
