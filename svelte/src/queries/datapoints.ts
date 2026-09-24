import type { Datapoint } from '@/lib/datapoints';
import { vars, type Transform } from 'crelte/queries';

export const variables = {
	id: vars.id(),
	siteId: vars.siteId(),
};

export const transform: Transform<typeof variables> = resp => {
	const datapoints = resp.datapoints.map(transformDatapoint);
	datapoints.sort((a, b) => a.minutes - b.minutes);

	return { datapoints };
};

function transformDatapoint(data: any): Datapoint {
	const price = parseFloat(data.price) || 0;
	const wage = parseFloat(data.wage) || 0;
	const minutes = wage > 0 ? (price / wage) * 60 : 0;

	return {
		country: data.country[0]?.title,
		price: formatPrice(price),
		wage: formatWage(wage),
		minutes,
		time: formatMinute(minutes),
	};
}

function formatPrice(price: number): string {
	return `$${price.toFixed(2)}`;
}

function formatWage(wage: number): string {
	return `$${wage.toFixed(2)}/hr`;
}

function formatMinute(minutes: number): string {
	return `${minutes.toFixed(2)}m`;
}
