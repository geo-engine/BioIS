import { ResponseContext, RequestContext, HttpFile, HttpInfo } from '../http/http';
import { Configuration, PromiseConfigurationOptions, wrapOptions } from '../configuration'
import { PromiseMiddleware, Middleware, PromiseMiddlewareWrapper } from '../middleware';

import { AdditionalParameter } from '../models/AdditionalParameter';
import { AdditionalParameters } from '../models/AdditionalParameters';
import { AuthCodeResponse } from '../models/AuthCodeResponse';
import { BoundingBox } from '../models/BoundingBox';
import { Conformance } from '../models/Conformance';
import { DescriptionType } from '../models/DescriptionType';
import { Exception } from '../models/Exception';
import { Execute } from '../models/Execute';
import { Format } from '../models/Format';
import { InlineOrRefData } from '../models/InlineOrRefData';
import { Input } from '../models/Input';
import { InputDescription } from '../models/InputDescription';
import { InputValue } from '../models/InputValue';
import { JobControlOptions } from '../models/JobControlOptions';
import { JobList } from '../models/JobList';
import { JobType } from '../models/JobType';
import { LandingPage } from '../models/LandingPage';
import { Link } from '../models/Link';
import { MaxOccurs } from '../models/MaxOccurs';
import { Metadata } from '../models/Metadata';
import { NDVIProcessInputs } from '../models/NDVIProcessInputs';
import { NDVIProcessOutputs } from '../models/NDVIProcessOutputs';
import { NDVIProcessParams } from '../models/NDVIProcessParams';
import { Output } from '../models/Output';
import { OutputDescription } from '../models/OutputDescription';
import { PointGeoJson } from '../models/PointGeoJson';
import { PointGeoJsonInput } from '../models/PointGeoJsonInput';
import { PointGeoJsonInputMediaType } from '../models/PointGeoJsonInputMediaType';
import { PointGeoJsonType } from '../models/PointGeoJsonType';
import { Process } from '../models/Process';
import { ProcessList } from '../models/ProcessList';
import { ProcessSummary } from '../models/ProcessSummary';
import { QualifiedInputValue } from '../models/QualifiedInputValue';
import { Response } from '../models/Response';
import { Results } from '../models/Results';
import { Schema } from '../models/Schema';
import { StatusCode } from '../models/StatusCode';
import { StatusInfo } from '../models/StatusInfo';
import { Subscriber } from '../models/Subscriber';
import { TransmissionMode } from '../models/TransmissionMode';
import { UserInfo } from '../models/UserInfo';
import { UserSession } from '../models/UserSession';
import { ObservableCapabilitiesApi } from './ObservableAPI';

import { CapabilitiesApiRequestFactory, CapabilitiesApiResponseProcessor} from "../apis/CapabilitiesApi";
export class PromiseCapabilitiesApi {
    private api: ObservableCapabilitiesApi

    public constructor(
        configuration: Configuration,
        requestFactory?: CapabilitiesApiRequestFactory,
        responseProcessor?: CapabilitiesApiResponseProcessor
    ) {
        this.api = new ObservableCapabilitiesApi(configuration, requestFactory, responseProcessor);
    }

    /**
     * API definition
     */
    public apiWithHttpInfo(_options?: PromiseConfigurationOptions): Promise<HttpInfo<{ [key: string]: any; }>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.apiWithHttpInfo(observableOptions);
        return result.toPromise();
    }

    /**
     * API definition
     */
    public api_(_options?: PromiseConfigurationOptions): Promise<{ [key: string]: any; }> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.api(observableOptions);
        return result.toPromise();
    }

    /**
     * A list of all conformance classes specified in a standard that the server conforms to.
     * API conformance definition
     */
    public conformanceWithHttpInfo(_options?: PromiseConfigurationOptions): Promise<HttpInfo<Conformance>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.conformanceWithHttpInfo(observableOptions);
        return result.toPromise();
    }

    /**
     * A list of all conformance classes specified in a standard that the server conforms to.
     * API conformance definition
     */
    public conformance(_options?: PromiseConfigurationOptions): Promise<Conformance> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.conformance(observableOptions);
        return result.toPromise();
    }

    /**
     * The landing page provides links to the API definition and the conformance statements for this API.
     * Landing page
     */
    public rootWithHttpInfo(_options?: PromiseConfigurationOptions): Promise<HttpInfo<LandingPage>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.rootWithHttpInfo(observableOptions);
        return result.toPromise();
    }

    /**
     * The landing page provides links to the API definition and the conformance statements for this API.
     * Landing page
     */
    public root(_options?: PromiseConfigurationOptions): Promise<LandingPage> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.root(observableOptions);
        return result.toPromise();
    }


}



import { ObservableDefaultApi } from './ObservableAPI';

import { DefaultApiRequestFactory, DefaultApiResponseProcessor} from "../apis/DefaultApi";
export class PromiseDefaultApi {
    private api: ObservableDefaultApi

    public constructor(
        configuration: Configuration,
        requestFactory?: DefaultApiRequestFactory,
        responseProcessor?: DefaultApiResponseProcessor
    ) {
        this.api = new ObservableDefaultApi(configuration, requestFactory, responseProcessor);
    }

    /**
     */
    public healthHandlerWithHttpInfo(_options?: PromiseConfigurationOptions): Promise<HttpInfo<void>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.healthHandlerWithHttpInfo(observableOptions);
        return result.toPromise();
    }

    /**
     */
    public healthHandler(_options?: PromiseConfigurationOptions): Promise<void> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.healthHandler(observableOptions);
        return result.toPromise();
    }


}



import { ObservableProcessesApi } from './ObservableAPI';

import { ProcessesApiRequestFactory, ProcessesApiResponseProcessor} from "../apis/ProcessesApi";
export class PromiseProcessesApi {
    private api: ObservableProcessesApi

    public constructor(
        configuration: Configuration,
        requestFactory?: ProcessesApiRequestFactory,
        responseProcessor?: ProcessesApiResponseProcessor
    ) {
        this.api = new ObservableProcessesApi(configuration, requestFactory, responseProcessor);
    }

    /**
     * Cancel a job execution and remove it from the jobs list.  For more information, see [Section 13](https://docs.ogc.org/is/18-062/18-062.html#Dismiss).
     * Cancel a job execution, remove finished job
     * @param jobId
     */
    public _deleteWithHttpInfo(jobId: string, _options?: PromiseConfigurationOptions): Promise<HttpInfo<StatusInfo>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api._deleteWithHttpInfo(jobId, observableOptions);
        return result.toPromise();
    }

    /**
     * Cancel a job execution and remove it from the jobs list.  For more information, see [Section 13](https://docs.ogc.org/is/18-062/18-062.html#Dismiss).
     * Cancel a job execution, remove finished job
     * @param jobId
     */
    public _delete(jobId: string, _options?: PromiseConfigurationOptions): Promise<StatusInfo> {
        const observableOptions = wrapOptions(_options);
        const result = this.api._delete(jobId, observableOptions);
        return result.toPromise();
    }

    /**
     * @param nDVIProcessParams
     */
    public executeNdviWithHttpInfo(nDVIProcessParams: NDVIProcessParams, _options?: PromiseConfigurationOptions): Promise<HttpInfo<NDVIProcessOutputs>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.executeNdviWithHttpInfo(nDVIProcessParams, observableOptions);
        return result.toPromise();
    }

    /**
     * @param nDVIProcessParams
     */
    public executeNdvi(nDVIProcessParams: NDVIProcessParams, _options?: PromiseConfigurationOptions): Promise<NDVIProcessOutputs> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.executeNdvi(nDVIProcessParams, observableOptions);
        return result.toPromise();
    }

    /**
     * Create a new job.  For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).
     * Execute a process
     * @param processID
     * @param execute
     */
    public executionWithHttpInfo(processID: string, execute: Execute, _options?: PromiseConfigurationOptions): Promise<HttpInfo<Results>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.executionWithHttpInfo(processID, execute, observableOptions);
        return result.toPromise();
    }

    /**
     * Create a new job.  For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).
     * Execute a process
     * @param processID
     * @param execute
     */
    public execution(processID: string, execute: Execute, _options?: PromiseConfigurationOptions): Promise<Results> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.execution(processID, execute, observableOptions);
        return result.toPromise();
    }

    /**
     * For more information, see [Section 11](https://docs.ogc.org/is/18-062/18-062.html#sc_job_list).
     * Retrieve the list of jobs
     * @param [limit] Amount of items to return
     * @param [offset] Offset into the items list
     */
    public jobsWithHttpInfo(limit?: number, offset?: number, _options?: PromiseConfigurationOptions): Promise<HttpInfo<JobList>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.jobsWithHttpInfo(limit, offset, observableOptions);
        return result.toPromise();
    }

    /**
     * For more information, see [Section 11](https://docs.ogc.org/is/18-062/18-062.html#sc_job_list).
     * Retrieve the list of jobs
     * @param [limit] Amount of items to return
     * @param [offset] Offset into the items list
     */
    public jobs(limit?: number, offset?: number, _options?: PromiseConfigurationOptions): Promise<JobList> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.jobs(limit, offset, observableOptions);
        return result.toPromise();
    }

    /**
     * The process description contains information about inputs and outputs and a link to the execution-endpoint for the process. The Core does not mandate the use of a specific process description to specify the interface of a process. That said, the Core requirements class makes the following recommendation:  Implementations SHOULD consider supporting the OGC process description.  For more information, see Section 7.10.
     * Retrieve a processes description
     * @param processID
     */
    public processWithHttpInfo(processID: string, _options?: PromiseConfigurationOptions): Promise<HttpInfo<Process>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.processWithHttpInfo(processID, observableOptions);
        return result.toPromise();
    }

    /**
     * The process description contains information about inputs and outputs and a link to the execution-endpoint for the process. The Core does not mandate the use of a specific process description to specify the interface of a process. That said, the Core requirements class makes the following recommendation:  Implementations SHOULD consider supporting the OGC process description.  For more information, see Section 7.10.
     * Retrieve a processes description
     * @param processID
     */
    public process(processID: string, _options?: PromiseConfigurationOptions): Promise<Process> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.process(processID, observableOptions);
        return result.toPromise();
    }

    /**
     * The list of processes contains a summary of each process the OGC API - Processes offers, including the link to a more detailed description of the process.  For more information, see [Section 7.9](https://docs.ogc.org/is/18-062/18-062.html#sc_process_list).
     * Retrieve the list of available processes
     */
    public processesWithHttpInfo(_options?: PromiseConfigurationOptions): Promise<HttpInfo<ProcessList>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.processesWithHttpInfo(observableOptions);
        return result.toPromise();
    }

    /**
     * The list of processes contains a summary of each process the OGC API - Processes offers, including the link to a more detailed description of the process.  For more information, see [Section 7.9](https://docs.ogc.org/is/18-062/18-062.html#sc_process_list).
     * Retrieve the list of available processes
     */
    public processes(_options?: PromiseConfigurationOptions): Promise<ProcessList> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.processes(observableOptions);
        return result.toPromise();
    }

    /**
     * Lists available results of a job. In case of a failure, lists exceptions instead.  For more information, see [Section 7.13](https://docs.ogc.org/is/18-062r2/18-062r2.html#sc_retrieve_job_results).  
     * Retrieve the result(s) of a job
     * @param jobId
     */
    public resultsWithHttpInfo(jobId: string, _options?: PromiseConfigurationOptions): Promise<HttpInfo<Results>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.resultsWithHttpInfo(jobId, observableOptions);
        return result.toPromise();
    }

    /**
     * Lists available results of a job. In case of a failure, lists exceptions instead.  For more information, see [Section 7.13](https://docs.ogc.org/is/18-062r2/18-062r2.html#sc_retrieve_job_results).  
     * Retrieve the result(s) of a job
     * @param jobId
     */
    public results(jobId: string, _options?: PromiseConfigurationOptions): Promise<Results> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.results(jobId, observableOptions);
        return result.toPromise();
    }

    /**
     * Shows the status of a job.  For more information, see [Section 7.12](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_status_info).
     * Retrieve the status of a job
     * @param jobId
     */
    public statusWithHttpInfo(jobId: string, _options?: PromiseConfigurationOptions): Promise<HttpInfo<StatusInfo>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.statusWithHttpInfo(jobId, observableOptions);
        return result.toPromise();
    }

    /**
     * Shows the status of a job.  For more information, see [Section 7.12](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_status_info).
     * Retrieve the status of a job
     * @param jobId
     */
    public status(jobId: string, _options?: PromiseConfigurationOptions): Promise<StatusInfo> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.status(jobId, observableOptions);
        return result.toPromise();
    }


}



import { ObservableUserApi } from './ObservableAPI';

import { UserApiRequestFactory, UserApiResponseProcessor} from "../apis/UserApi";
export class PromiseUserApi {
    private api: ObservableUserApi

    public constructor(
        configuration: Configuration,
        requestFactory?: UserApiRequestFactory,
        responseProcessor?: UserApiResponseProcessor
    ) {
        this.api = new ObservableUserApi(configuration, requestFactory, responseProcessor);
    }

    /**
     * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
     * @param authCodeResponse
     */
    public authHandlerWithHttpInfo(redirectUri: string, authCodeResponse: AuthCodeResponse, _options?: PromiseConfigurationOptions): Promise<HttpInfo<UserSession>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.authHandlerWithHttpInfo(redirectUri, authCodeResponse, observableOptions);
        return result.toPromise();
    }

    /**
     * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
     * @param authCodeResponse
     */
    public authHandler(redirectUri: string, authCodeResponse: AuthCodeResponse, _options?: PromiseConfigurationOptions): Promise<UserSession> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.authHandler(redirectUri, authCodeResponse, observableOptions);
        return result.toPromise();
    }

    /**
     * Generates a URL for initiating the OIDC code flow, which the frontend can use to redirect the user to the identity provider\'s login page.
     * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
     */
    public authRequestUrlHandlerWithHttpInfo(redirectUri: string, _options?: PromiseConfigurationOptions): Promise<HttpInfo<string>> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.authRequestUrlHandlerWithHttpInfo(redirectUri, observableOptions);
        return result.toPromise();
    }

    /**
     * Generates a URL for initiating the OIDC code flow, which the frontend can use to redirect the user to the identity provider\'s login page.
     * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
     */
    public authRequestUrlHandler(redirectUri: string, _options?: PromiseConfigurationOptions): Promise<string> {
        const observableOptions = wrapOptions(_options);
        const result = this.api.authRequestUrlHandler(redirectUri, observableOptions);
        return result.toPromise();
    }


}



