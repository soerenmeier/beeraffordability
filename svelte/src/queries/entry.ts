import { processCke } from '@/lib/queriesUtils';

export function transform(resp: any) {
	const entry = resp.entry;
	if (!entry) return;

	processCke(entry.pageIntroCke);
	processCke(entry.ctnCke);
}
