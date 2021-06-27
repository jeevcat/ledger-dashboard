import React, { useCallback, useContext, useEffect, useState } from "react";
import { Button, Container, Icon, Loader } from "semantic-ui-react";
import { Rule } from "../../Models/Rule";
import { AccountsContext } from "../../Utils/AccountsContext";
import { deleteRule, getRules, setRule } from "../../Utils/BackendRequester";
import RulesTable from "./RulesTable";

interface Props {}

type RuleErrors = { [rule: number]: string | undefined };

const Rules: React.FC<Props> = () => {
  const [isLoadingRules, setIsLoadingRules] = useState(false);
  const [rules, setRules] = useState<Rule[]>([]);
  const [errors, setErrors] = useState<RuleErrors>({});

  const { importAccount } = useContext(AccountsContext);

  const handleRuleSave = async (rule: Rule): Promise<boolean> => {
    return setRule(importAccount, rule).then((error) => {
      if (error) {
        setErrors((prevErrors) => ({ ...prevErrors, [rule.id]: error }));
        return true;
      } else {
        updateRules();
        return false;
      }
    });
  };

  const handleRuleDelete = async (rule: Rule) => {
    deleteRule(rule).then(() => {
      updateRules();
    });
  };

  const handleRuleNew = () => {
    const rule: Rule = {
      id: 0,
      priority: 100,
      importerId: importAccount.id,
      ruleName: "NEW RULE",
      matchFieldName: "description",
      descriptionTemplate: "{{description}}",
      matchFieldRegex: "(?i)",
      postings: [
        {
          account: "?",
          negate: false,
        },
      ],
    };
    setRule(importAccount, rule).then(updateRules);
  };

  const fetchRules = useCallback(() => {
    setIsLoadingRules(true);
    getRules(importAccount)
      .then((data: Rule[]) => {
        setRules(data);
        setIsLoadingRules(false);
      })
      .catch((e) => console.error(`Couldn't fetch rules ${e}`));
  }, [importAccount]);

  const updateRules = () => {
    fetchRules();
  };

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

  if (isLoadingRules) {
    return <Loader active />;
  } else {
    return (
      <>
        <RulesTable
          errors={errors}
          rules={rules}
          onSetRuleRequested={handleRuleSave}
          onDeleteRuleRequested={handleRuleDelete}
        />
        <br />
        <Container fluid textAlign="right">
          <Button primary icon labelPosition="right" onClick={handleRuleNew}>
            New
            <Icon name="add" />
          </Button>
        </Container>
      </>
    );
  }
};

export default Rules;
