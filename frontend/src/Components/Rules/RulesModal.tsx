import React, { useState, useEffect } from "react";
import { Button, Icon, Loader, Modal } from "semantic-ui-react";
import { Rule } from "../../Models/Rule";
import { getRules, deleteRule, setRule } from "../../Utils/BackendRequester";
import RulesTable from "./RulesTable";
import { RealTransactionField } from "../../Models/ImportRow";

interface Props {
  onRulesFetched: () => void;
  realTransactionFields: RealTransactionField[];
  accounts: string[];
}

const NUMBER_FIELDS: (keyof Rule)[] = ["priority"];

type RuleErrors = { [rule: number]: string | undefined };

const RulesModal: React.FC<Props> = ({ onRulesFetched, realTransactionFields, accounts }) => {
  const [rulesOpen, setRulesOpen] = useState(false);
  const [isLoadingRules, setIsLoadingRules] = useState(false);
  const [dirtyRules, setDirtyRules] = useState<number[]>([]);
  const [rules, setRules] = useState<Rule[]>([]);
  const [errors, setErrors] = useState<RuleErrors>({});

  // Don't fully get this syntax yet: https://github.com/microsoft/TypeScript/issues/24197#issuecomment-389928513
  const handleRuleEdit = <K extends keyof Rule>(id: number, field: K, value: any) => {
    setDirtyRules((prevDirtyRules) => {
      return [...prevDirtyRules.filter((prevId) => prevId !== id), id];
    });
    setRules((prevRules) => {
      const rule = prevRules.find((r) => r.id === id);
      if (rule === undefined) {
        return prevRules;
      }
      let newValue = value;
      if (NUMBER_FIELDS.includes(field)) {
        newValue = parseInt(value) || value;
      }
      return prevRules.map((r) => {
        return r.id === rule.id ? { ...rule, [field]: newValue } : r;
      });
    });
  };

  const handleRuleSave = () => {
    const newErrors: RuleErrors = {};
    Promise.all(
      dirtyRules.map((id) => {
        const rule = rules.find((r) => r.id === id)!;
        return setRule(rule).then((error) => {
          if (error) {
            newErrors[id] = error;
          }
        });
      })
    ).then(() => {
      setDirtyRules([]);
      if (Object.keys(newErrors).length > 0) {
        setErrors(newErrors);
      } else {
        updateRules();
      }
    });
  };

  const handleRuleDelete = (rule: Rule) => {
    deleteRule(rule).then(() => {
      updateRules();
    });
  };

  const fetchRules = () => {
    setIsLoadingRules(true);
    getRules().then((data: Rule[]) => {
      setRules(data);
      setIsLoadingRules(false);
    });
  };

  const updateRules = () => {
    fetchRules();
    onRulesFetched();
  };

  useEffect(() => {
    fetchRules();
  }, []);

  return (
    <Modal
      size="large"
      onClose={() => setRulesOpen(false)}
      onOpen={() => setRulesOpen(true)}
      open={rulesOpen}
      trigger={<Button>Edit Rules</Button>}
    >
      <Modal.Header>Rules</Modal.Header>
      <Modal.Content>
        {isLoadingRules ? (
          <Loader active />
        ) : (
          <RulesTable
            accounts={accounts}
            errors={errors}
            ruleFields={realTransactionFields}
            rules={rules}
            onDeleteRuleRequested={handleRuleDelete}
            onEditRuleRequested={handleRuleEdit}
            onUpdateNeeded={updateRules}
          />
        )}
      </Modal.Content>
      <Modal.Actions>
        <Button color="red" onClick={() => setRulesOpen(false)}>
          <Icon name="remove" /> Cancel
        </Button>
        <Button
          color="green"
          disabled={dirtyRules.length === 0}
          onClick={() => {
            handleRuleSave();
            setRulesOpen(false);
          }}
        >
          <Icon name="save" /> Save
        </Button>
      </Modal.Actions>
    </Modal>
  );
};

export default RulesModal;
