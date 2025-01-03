import { execSync } from "child_process";
import { DocumentSemanticTokensProvider, ExtensionContext, Position, ProviderResult, Range, SemanticTokens, SemanticTokensBuilder, SemanticTokensLegend, TextDocument, languages, window } from "vscode";

type SemRule = keyof typeof RuleMap;
interface SemPage {
	page: string,
	tokens: SemToken[]
};

interface SemToken {
	rule: SemRule,
	span: [[number, number], [number, number]]
};

type SemAny = SemPage[];

const RuleMap = {
	'uses': 'keyword',
	'export': 'keyword',
	'from': 'keyword',

	'string': 'string',
	'number': 'number',

	'pow': 'operator',
	'equality': 'operator',
	'add': 'operator',
	'subtract': 'operator',
	'multiply': 'operator',
	'divide': 'operator',
	'or': 'operator',
	'and': 'operator',
	'gt': 'operator',
	'lt': 'operator',
	'gte': 'operator',
	'lte': 'operator',

	'index': 'property',
	'identifier': 'variable',
};

export const legend = new SemanticTokensLegend(Object.values(RuleMap).filter((v, i, a) => a.indexOf(v) === i), ['declaration']);
export const provider: DocumentSemanticTokensProvider = {
	provideDocumentSemanticTokens(
		document: TextDocument
	): ProviderResult<SemanticTokens> {
		const tokensBuilder = new SemanticTokensBuilder(legend);
		const res: SemAny = JSON.parse(execSync(`flang --semantic-analysis ${document.uri.fsPath}`).toString());

		// document.getText().split('\n').map((text, line) => text.split('').forEach((char, col) => {
		// 	if (['(', ')', '{', '}'].includes(char)) {

		// 	}
		// }));

		for (const token of res.find(p => p.page === document.uri.fsPath)!.tokens) {
			if (!RuleMap[token.rule]) { continue; }
			console.log(token.rule);

			tokensBuilder.push(
				new Range(new Position(token.span[0][0] - 1, token.span[0][1] - 1), new Position(token.span[1][0] - 1, token.span[1][1] - 1)),
				RuleMap[token.rule],
				[]
			);
		}


		return tokensBuilder.build();
	}
};

export function activate(context: ExtensionContext) {
	// console.log(Object.values(RuleMap).filter((v, i, a) => a.indexOf(v) === i));
	// context.subscriptions.push(languages.registerDocumentSemanticTokensProvider({ language: 'flang' }, provider, legend));
}