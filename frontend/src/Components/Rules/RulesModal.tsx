import React, { useCallback, useContext, useEffect, useState } from "react";
import { Button, Icon, Loader, Modal } from "semantic-ui-react";
import { RealTransactionField } from "../../Models/ImportRow";
import { Rule } from "../../Models/Rule";
import { AccountsContext } from "../../Utils/AccountsContext";
import { deleteRule, getRules, setRule } from "../../Utils/BackendRequester";
import RulesTable from "./RulesTable";

interface Props {
  onRulesChanged: () => void;
  realTransactionFields: RealTransactionField[];
}

const NUMBER_FIELDS: (keyof Rule)[] = ["priority"];

type RuleErrors = { [rule: number]: string | undefined };

const RulesModal: React.FC<Props> = ({ onRulesChanged, realTransactionFields }) => {
  const [rulesOpen, setRulesOpen] = useState(false);
  const [isLoadingRules, setIsLoadingRules] = useState(false);
  const [dirtyRules, setDirtyRules] = useState<number[]>([]);
  const [rules, setRules] = useState<Rule[]>([]);
  const [errors, setErrors] = useState<RuleErrors>({});

  const { importAccount } = useContext(AccountsContext);

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

  const handleRuleSave = async (): Promise<boolean> => {
    const newErrors: RuleErrors = {};
    return await Promise.all(
      dirtyRules.map((id) => {
        const rule = rules.find((r) => r.id === id)!;
        return setRule(importAccount, rule).then((error) => {
          if (error) {
            newErrors[id] = error;
          }
        });
      })
    ).then(() => {
      setDirtyRules([]);
      if (Object.keys(newErrors).length > 0) {
        setErrors(newErrors);
        return true;
      } else {
        setErrors({});
        updateRules();
        return false;
      }
    });
  };

  const handleRuleDelete = (rule: Rule) => {
    deleteRule(rule).then(() => {
      updateRules();
    });
  };

  const handleRuleNew = () => {
    const rule: Rule = {
      id: 0,
      priority: 100,
      ruleName: "NEW RULE",
      matchFieldName: "description",
      targetAccount: "?",
      descriptionTemplate: "{{description}}",
      matchFieldRegex: "$^",
    };
    setRule(importAccount, rule).then(updateRules);
  };

  const fetchRules = useCallback(() => {
    setIsLoadingRules(true);
    getRules(importAccount).then((data: Rule[]) => {
      setRules(data);
      setIsLoadingRules(false);
    });
  }, [importAccount]);

  const updateRules = () => {
    fetchRules();
    onRulesChanged();
  };

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

  return (
    <Modal
      size="fullscreen"
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
            errors={errors}
            ruleFields={realTransactionFields}
            rules={rules}
            onDeleteRuleRequested={handleRuleDelete}
            onEditRuleRequested={handleRuleEdit}
            onNewRuleRequested={handleRuleNew}
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
            handleRuleSave().then(setRulesOpen);
          }}
        >
          <Icon name="save" /> Save
        </Button>
      </Modal.Actions>
    </Modal>
  );
};

export default RulesModal;
