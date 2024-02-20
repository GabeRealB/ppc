import { createSyntheticDataset } from './datasets/synthetic';
import { createAdultDataset } from './datasets/adult';
import { createAblationDataset } from './datasets/ablation';
import { Props } from 'types';

export function syntheticDataset(visible: string[], include: string[]): Props {
    const included = new Set([...visible, ...include]);
    const dataset = createSyntheticDataset(Array.from(included));
    return {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
}

export function adultDataset(visible: string[], include: string[], samples?: number): Props {
    const included = new Set([...visible, ...include]);
    const dataset = createAdultDataset(Array.from(included), samples);
    return {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
}

export function ablationDataset(visible: string[], include: string[], samples?: number): Props {
    const included = new Set([...visible, ...include]);
    const dataset = createAblationDataset(Array.from(included), samples);
    return {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
}