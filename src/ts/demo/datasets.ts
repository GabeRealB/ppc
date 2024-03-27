import { createSyntheticTestDataset } from './datasets/synthetic_test';
import { createSyntheticDataset } from './datasets/synthetic';
import { createAdultDataset } from './datasets/adult';
import { createAblationDataset } from './datasets/ablation';

import { createIrisDataset } from './datasets/iris';
import { createShipDataset } from './datasets/ship';

import { Props } from 'types';

export function syntheticTestDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createSyntheticTestDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function syntheticDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createSyntheticDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function adultDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createAdultDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function ablationDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createAblationDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function irisDataset(): Props {
    const dataset = createIrisDataset();
    const state = {
        axes: dataset,
        order: [],
        labels: {},
        setProps: undefined
    };
    return state;
}

export function shipDataset(): Props {
    const dataset = createShipDataset();
    const state = {
        axes: dataset,
        order: [],
        labels: {},
        setProps: undefined
    };
    return state;
}