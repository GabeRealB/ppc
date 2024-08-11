import { createSyntheticTestDataset } from './datasets/synthetic_test';
import { createApplicationDataset } from './datasets/application';
import { createValidationDataset } from './datasets/validation';
import { createDerivationDataset } from './datasets/derivation';
import { createDerivationControlDataset } from './datasets/derivation_control';

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

export function applicationDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createApplicationDataset(Array.from(included), samples);
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

export function validationDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createValidationDataset(Array.from(included), samples);
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

export function derivationDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createDerivationDataset(Array.from(included), samples);
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

export function derivationControlDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createDerivationControlDataset(Array.from(included), samples);
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