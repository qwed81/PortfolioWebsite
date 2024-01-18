// @deno-types="npm:@types/express@4.17.2"
import express from "npm:express@4.18.2";
import _ from "npm:ejs@3.1.9";

const app = express();

app.set('view engine', 'ejs');
app.use('/static', express.static(Deno.cwd() + '/static'));

app.get('/', async (req, res) => {
	await serve_article('article-list', res);
});

app.get('/:article', async (req, res) => {
	await serve_article(req.params.article, res);
});

async function serve_article(name: string, res: any) {
	let text: string;
		
	try {
		text = await Deno.readTextFile(`articles/${name}`); 
	} catch {
		res.status(404).send('page not found');
		return;
	}

	const tryArticle: Article | Error = parseArticle(text);
	let article: Article;
	if (tryArticle instanceof Error) {
		res.status(500).send(tryArticle.toString());
		return;
	} else {
		article = tryArticle;
	}

	res.render('article', article);
}


app.listen(8080, () => {
	console.log('listening on port: 8080');
});

type SectionType = 'title' | 'p' | 'code' | 'img';

interface Section {
	type: SectionType;
	data: string; // url or text depending on type
};

interface Article {
	sections: Section[];
	title: string
}

function parseArticle(text: string): Article | Error {
	const sections = parseSection(text, 0);
	if (sections instanceof Error) {
		return sections;
	}

	let title: string;
	if (sections.length < 1 || sections[0].type != 'title') {
		return new Error('requires main heading');
	} else {
		title = sections.shift()!.data;
	}

	return {
		sections,
		title,
	};
}

function parseSection(text: string, start: number): Section[] | Error {
	if (start < 0 || start >= text.length) {
		return new Error('out of bounds start');
	}

	if (text.substring(start, start + 3) !== '--?') {
		return new Error('expected section identifier');
	}

	const sectionText = text.substring(start + 3);
	let sectionType: SectionType;

	let dataStart: number;
	if (sectionText.startsWith('title\n')) {
		sectionType = 'title';
		dataStart = 6;
	} else if (sectionText.startsWith('p\n')) {
		sectionType = 'p';
		dataStart = 2;
	} else if (sectionText.startsWith('code\n')) {
		sectionType = 'code';
		dataStart = 5;
	} else if (sectionText.startsWith('img\n')) {
		sectionType = 'img';
		dataStart = 4;
	} else {
		return new Error('unsupported section identifier');
	}

	const nextIndex = sectionText.indexOf('--?');
	if (nextIndex === -1) {
		const thisSection: Section = {
			type: sectionType,
			data: sectionText.substring(dataStart)
		};
		return [thisSection];
	}

	const nextSections = parseSection(sectionText, nextIndex);
	if (nextSections instanceof Error) {
		return nextSections;
	} 

	const thisSection: Section = {
		type: sectionType,
		data: sectionText.substring(dataStart, nextIndex)
	};
	nextSections.unshift(thisSection);
	return nextSections;
}

