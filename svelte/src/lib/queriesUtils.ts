function processChunks(chunks: null | any[]) {
	if (!chunks) return [];

	const processedChunks: any[] = [];

	for (const chunk of chunks) {
		if (chunk.html) {
			processedChunks.push(chunk);
		} else if (chunk.btn?.url) {
			const previous = processedChunks[processedChunks.length - 1];

			if (previous?.btns) {
				previous.btns.push(chunk.btn);
			} else {
				processedChunks.push({ btns: [chunk.btn] });
			}
		}
	}

	return processedChunks;
}

export function processCke(cke: any) {
	if (!cke) return;

	cke.chunks = processChunks(cke.chunks);
}
